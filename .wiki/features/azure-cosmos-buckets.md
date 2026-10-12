---
okf_version: "0.1"
type: Plan
title: Azure Cosmos DB bucket downloads
description: Treat an Azure Cosmos DB for NoSQL container or logical partition as a read-only ACORN bucket source.
difficulty: high
impact: high
package:
  - workspace
  - acorn-lib
  - acorn-cli
tags:
  - azure
  - bucket
  - cosmos-db
  - download
  - provider
status: planned
execution: phased
phase_budget_hours: 5
---

# Project code boundaries

- **Update:** workspace and `acorn-lib` manifests and lockfile; `crates/acorn-lib/src/io/config/{bucket/mod.rs,bucket/source.rs,mod.rs}`; focused API/authentication integration; `crates/acorn-lib/src/io/config/tests/`; `crates/acorn-cli/src/commands/download/`; dedicated configuration and Cosmos fixtures; download and configuration documentation; generated CLI help only if user-facing arguments change.
- **Create:** a focused Cosmos bucket client/adapter leaf under `crates/acorn-lib/src/io/api/`; provider-specific tests and fixtures under existing test roots.
- **Delete:** None.

# Dependencies

This plan requires the implemented provider-neutral `BucketSource` and `BucketSourceAdapter` contracts and the item-aware manifest, filtering, destination-safety, canonicalization, provenance, and ingestion pipeline reused by the landed BagIt archive import and download path. Cosmos integration must extend the landed source and item contracts rather than revise them in the provider plan.

It must preserve the landed dependency capability boundary, the existing HTTP/offline policy, and the provider/authentication boundary in the [shared API client contract](../refactors/api/shared-client-contract.md).

The first release uses Azure Cosmos DB for NoSQL. Other Cosmos APIs, writes, change-feed synchronization, and database-backend replacement are outside this plan.

# Goal

Add one read-only Cosmos variant and adapter to the provider-neutral bucket source contract, allowing a configured ACORN bucket to resolve from an Azure Cosmos DB for NoSQL container or one logical partition and pass retrieved RAD documents through the existing transfer pipeline.

# Current evidence

- `Bucket` now contains a non-exhaustive `BucketSource`; `RepositoryBucketSource` implements `BucketSourceAdapter`, and `transfer_bucket_files` owns provider-neutral filtering and transfer work.
- `TransferManifest` owns ordered source keys, source and output paths, exact versions, sizes, post-transfer canonicalization, canonical candidate ingestion, and versioned bucket provenance. A Cosmos provider does not need a parallel validation or persistence pipeline.
- Repository adapters resolve immutable GitHub and GitLab versions before listing and use those versions for retrieval, establishing the consistency contract expected of new remote adapters.
- `Repository` in `acorn-core` still models Git, GitHub, GitLab, Hugging Face, and OCI locations. The landed abstraction keeps that enum unchanged and establishes `BucketSource` as the extension point for Cosmos.
- Cosmos DB for NoSQL stores JSON items and supports point reads and parameterized queries. Transactions and transactional batches are scoped to one logical partition, so container and partition-key design remain explicit even for a read-only first release ([Azure Cosmos DB transactions](https://learn.microsoft.com/azure/cosmos-db/database-transactions-optimistic-concurrency), [transactional batches](https://learn.microsoft.com/azure/cosmos-db/transactional-batch)).
- The official Rust client is available as `azure_data_cosmos`, but Microsoft currently labels it public preview and does not recommend it for production workloads ([Azure SDK for Rust Cosmos quickstart](https://learn.microsoft.com/azure/cosmos-db/quickstart-rust)). The adapter boundary must keep the client replaceable and the dependency optional.

# Decisions

## Cosmos source configuration

Extend the landed `BucketSource` enum with one Cosmos variant. Preserve its repository configuration compatibility and accept a Cosmos form such as:

```json
{
  "buckets": [
    {
      "name": "nssd",
      "source": {
        "provider": "cosmos",
        "endpoint": "https://example.documents.azure.com",
        "database": "acorn",
        "container": "research-activities",
        "partition": "nssd"
      }
    }
  ]
}
```

A bucket maps to one container plus an optional logical-partition value. Do not interpret an entire Cosmos account or database as an implicit bucket, and do not permit credentials in application configuration. The Cosmos adapter lists `BucketSourceItem` values from bounded, paginated queries and uses point reads for selected items after shared filters run. Ordering must be deterministic before transfer planning.

## Cosmos item envelope

Use a versioned envelope so Cosmos system fields and partition metadata do not enter the downloaded RAD document:

```json
{
  "id": "project-a",
  "bucket": "nssd",
  "path": "project-a/index.json",
  "mediaType": "application/json",
  "schemaVersion": 1,
  "content": {
    "meta": {
      "identifier": "project-a",
      "title": "Project A"
    }
  }
}
```

The initial contract requires a safe relative `path`, JSON media type, supported envelope version, and object-valued `content`. Reject duplicate paths, unsafe paths, malformed envelopes, unsupported media, oversized content, and conflicting identifiers before writing output. Use `/bucket` as the documented default partition key while allowing an explicitly configured compatible partition field only if the implementation can point-read every listed item without a cross-partition scan.

Binary assets are outside the first release. Store large media in an object store and represent it through an existing supported RAD reference rather than base64-encoding it into Cosmos items.

## Authentication and transport

Use Microsoft Entra ID through the Azure identity credential chain by default. Keep account-key support out of the first release unless deployment evidence requires it and the existing `SecretString` contract can contain it without configuration-file or diagnostic exposure.

Honor `--offline` before constructing a client or issuing a request. Bound query pages, item count, response bytes, concurrency, retries, and total elapsed time. Handle continuation tokens and throttling without logging authorization material, item contents, or raw service responses. Preserve the shared redirect, proxy, TLS, and diagnostic-redaction policy where the SDK permits configuration; otherwise document and test the narrower SDK behavior.

## Feature and provenance boundary

Place Azure dependencies behind an optional `cosmos` capability that remains absent from portable, WASM, and unrelated host graphs. Record stable, credential-free provenance containing the configured bucket name, provider, account endpoint authority, database, container, partition selection, item ID, virtual path, observation time, and Cosmos item `_etag` when available.

Do not reinterpret Cosmos as `--database-backend`. SQLite and DuckDB remain ACORN's local cache/history stores.

# Recommendation

Implement the optional Cosmos client and envelope validation as a provider leaf, extend `BucketSource` with one explicit variant, and integrate it through the existing transfer executor without reopening repository adaptation or shared transfer policy.

# Implementation sequence

1. Add the optional Cosmos client leaf, `BucketSource` variant, typed configuration, envelope validation, paginated listing, selected-item point reads, deterministic ordering, and credential-free error mapping.
2. Integrate Cosmos items with the shared transfer executor and add focused CLI tests for configuration expansion, explicit output, filtering, flattening, clobbering, strict validation, canonicalization, local ingestion, quiet output, and offline rejection.
3. Document the envelope, partition-key requirement, identity configuration, optional feature, Rust SDK preview status, limits, and read-only scope. Regenerate help only when the CLI contract changes.

# Validation

- Re-run the landed abstraction's JSON, JSONC, YAML, repository-transfer, manifest, and provenance compatibility suite after registering the Cosmos variant.
- Unit-test Cosmos configuration validation, safe virtual paths, stable ordering, continuation-token pagination, duplicate paths, unsupported versions/media, missing partition data, malformed content, item and response bounds, throttling/retry exhaustion, redaction, and offline short-circuiting.
- Use a deterministic fake Cosmos transport for normal tests. Add an opt-in emulator or Azure integration test only when it can run without committed credentials and can verify identity, pagination, point reads, and failure mapping reliably.
- Exercise download filters, ignores, flattening collisions, clobbering, strict and canonicalized RAD handling, provenance, local-database ingestion, and `--no-local-database` through the production command path.
- Inspect default, Cosmos-only, all-feature, no-default-feature, analysis-only, and WASM dependency graphs. Run focused tests, `make lint`, `make test`, `make help` when needed, `make check-wasm`, and the accepted dependency/license audit.

# Acceptance criteria

- A user can configure a Cosmos container and logical partition as an ACORN bucket and download its valid RAD envelopes with the existing `acorn download --config` workflow.
- The Cosmos implementation adds only its source variant, adapter, configuration, provenance, fixtures, and integration to the landed provider-neutral bucket source contract.
- Existing repository-backed buckets retain their accepted legacy input aliases and observable transfer behavior; canonical serialization uses the provider-neutral `source` key.
- Cosmos virtual paths pass through the existing filter, ignore, flatten, destination-safety, clobber, canonicalization, provenance, and ingestion contracts rather than a parallel pipeline.
- Offline mode fails before Azure authentication or network access, and credentials and item contents do not appear in logs or errors.
- Pagination, deterministic ordering, duplicate and unsafe paths, malformed envelopes, unsupported media, response bounds, throttling, and partial failure have focused regression coverage.
- Azure dependencies are optional and absent from portable/WASM and unrelated feature graphs.
- The first release is read-only, JSON-only, and does not add Cosmos as a local database backend, upload destination, binary store, change-feed consumer, or generic provider registry.

# Execution phases

## COS-01 — Confirm demand and SDK feasibility

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: named Cosmos deployment consumer and accessible non-production container
- Exact write set: this plan, Cosmos feasibility evidence
- Primary integration lock: Cosmos adoption decision
- Outcome: Verify the preview SDK, license, MSRV, identity chain, transport controls, and required deployment shape before adding dependencies.
- Focused validation: Compile a minimal pinned client and record unsupported policy controls or stop adoption.
- Handoff: Remove COS-01 or retire the plan if demand/feasibility fails.

## COS-02 — Add the optional feature graph

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COS-01
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-lib/Cargo.toml`, `deny.toml`
- Primary integration lock: workspace manifests
- Outcome: Keep Azure dependencies behind `cosmos` and absent from portable, WASM, and unrelated graphs.
- Focused validation: Run dependency/license checks plus default, cosmos, no-default, and WASM builds.
- Handoff: Remove COS-02 and freeze the optional dependency boundary.

## COS-03 — Add typed Cosmos source configuration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COS-02
- Exact write set: bucket source configuration type, configuration tests
- Primary integration lock: bucket source schema
- Outcome: Add endpoint/database/container/partition configuration without credentials or repository-enum changes.
- Focused validation: Run serde, unknown-field, endpoint, partition, compatibility, and secret-rejection tests.
- Handoff: Remove COS-03 and expose the variant to the adapter.

## COS-04 — Validate the item envelope

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COS-03
- Exact write set: `crates/acorn-lib/src/io/cosmos.rs`, envelope tests
- Primary integration lock: Cosmos adapter
- Outcome: Enforce version, safe path, JSON media, object content, size, identifier, and duplicate-path rules.
- Focused validation: Run valid and every malformed/unsafe/conflicting envelope case.
- Handoff: Remove COS-04 and expose normalized source items.

## COS-05 — Implement bounded paginated listing

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COS-04
- Exact write set: `crates/acorn-lib/src/io/cosmos.rs`, listing tests
- Primary integration lock: Cosmos adapter
- Outcome: List deterministic source items through parameterized bounded pages and continuation tokens.
- Focused validation: Test ordering, pagination, item/page/byte/time ceilings, throttling, and redacted errors.
- Handoff: Remove COS-05 and expose selected item identities.

## COS-06 — Implement authenticated point reads

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COS-05
- Exact write set: `crates/acorn-lib/src/io/cosmos.rs`, point-read tests
- Primary integration lock: Cosmos adapter
- Outcome: Read selected items with Entra credentials, partition identity, retries, bounds, and early offline rejection.
- Focused validation: Test credential chain, not-found, ETag, partition mismatch, retry, oversize, timeout, and offline paths.
- Handoff: Remove COS-06 and publish the provider-neutral item stream.

## COS-07 — Integrate the shared transfer executor

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COS-06
- Exact write set: bucket transfer dispatch, Cosmos transfer tests
- Primary integration lock: bucket transfer service
- Outcome: Reuse filtering, flattening, clobbering, canonicalization, persistence, and manifest behavior.
- Focused validation: Run provider-neutral transfer cases against the Cosmos adapter.
- Handoff: Remove COS-07 and record shared-policy parity.

## COS-08 — Cover CLI configuration and filtering

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COS-07
- Exact write set: bucket download command tests, Cosmos configuration fixtures
- Primary integration lock: bucket CLI integration
- Outcome: Prove expansion, output, filters, flattening, clobbering, quiet output, and strict validation.
- Focused validation: Run focused compiled-command tests without public network access.
- Handoff: Remove COS-08 and preserve command fixtures.

## COS-09 — Persist bounded provenance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COS-07
- Exact write set: transfer provenance model, Cosmos provenance tests
- Primary integration lock: bucket transfer service
- Outcome: Record credential-free authority, database, container, partition, item, path, observation, and ETag data.
- Focused validation: Test serialization, redaction, canonical ingestion, replay, and partial failure.
- Handoff: Remove COS-09 and freeze provenance fields.

## COS-10 — Document the preview capability

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COS-08, COS-09
- Exact write set: bucket source documentation, `CommandLineHelp.md`
- Primary integration lock: bucket documentation
- Outcome: Document envelope, partition key, identity, limits, optional feature, preview status, and read-only scope.
- Focused validation: Run docs/help generation, examples, dependency-graph checks, and internal links.
- Handoff: Remove COS-10 and preserve the supported profile.

## COS-11 — Run opt-in live evidence and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COS-10
- Exact write set: non-production Cosmos container, this plan, wiki indexes
- Primary integration lock: Cosmos acceptance
- Outcome: Run the bounded live smoke, full offline gates, and remove the plan for the tested profile.
- Focused validation: Run live list/read/transfer, focused tests, feature/portable checks, `make lint`, `make test`, and wiki validation.
- Handoff: Delete this plan and every index, network, and lane entry.
