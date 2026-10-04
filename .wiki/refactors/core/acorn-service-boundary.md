---
okf_version: "0.1"
type: Plan
title: Extract portable analysis and service boundaries
description: Move supplied-content analysis and repository workflows into portable crates, then host them through JSON-RPC and typed gRPC in a lean acorn-server process.
difficulty: high
impact: critical
package: [workspace, acorn-analysis, acorn-service, acorn-server, acorn-lib, acorn-host, acorn-schema, acorn-wasm, acorn-cli]
tags: [refactor, analysis, service, server, json-rpc, grpc, portability]
status: planned
---

# Goal

Make ACORN straightforward to embed as a library or run as a small service without importing the current all-purpose `acorn-lib` host stack.

The first phase extracts `acorn-analysis` and `acorn-service` together so the service has an immediate portable consumer and the analysis boundary is proven by both native and WASM callers. The second phase adds `acorn-server`, a lean process that hosts the same application operations over authenticated JSON-RPC 2.0 and typed gRPC. Neither protocol is a compatibility facade over the other; both adapt the same immutable request values to the same service methods.

```text
supplied bytes/text
        |
        v
 acorn-analysis <--- acorn-schema
        |
        v
 acorn-service
    |         |
    v         v
 JSON-RPC   typed gRPC
       \     /
     acorn-server
```

This is a pre-release contract. Do not add a protocol version namespace, a version discovery operation, or compatibility shims. The plan establishes one current contract and updates all first-party consumers atomically.

# Current evidence

- `crates/acorn-lib/src/analyzer/` still combines portable check models, validation folds, FAIR/readability calculations, and supplied-data checks with filesystem traversal, document rendering, Vale execution, HTTP, and terminal presentation.
- The analyzer kernel already supports alloc-oriented and WASM builds in-tree, but `acorn-wasm` imports it through `acorn-lib` and independently maps readability and validation results into JavaScript DTOs.
- `OutputFormat`, host path discovery, document indexes, fixes, inference, and Vale runtime configuration depend on native execution or presentation and do not belong in a portable analysis crate.
- `crates/acorn-lib/src/io/workflow/registry.rs` already processes caller-supplied repository files into a deterministic `RepositoryChangeSet`, but it depends on `acorn-host::SafePath`, path-based `InputOutput` helpers, and formatting behavior that can inspect the current working directory for images.
- `OperationRegistry` already provides typed JSON-RPC registration, effect authorization, ordered batches, notifications, bounded outputs, durable status/replay operations, and standard JSON-RPC errors.
- The Axum adapter already exposes authenticated `POST /rpc`, checks `application/json`, caps request bodies, and returns no content for notification-only calls.
- The current gRPC surface is a generic `Invoke`/`Notify`/`Batch` envelope over `google.protobuf.Value`. Its tests prove authentication, deadlines, cancellation, and equivalence with direct and JSON-RPC dispatch, but the wire contract does not describe ACORN operations.
- `acorn serve rpc` and `acorn serve grpc` are separate CLI composition paths. JSON-RPC currently defaults to `127.0.0.1:8001`; gRPC defaults to `127.0.0.1:50051` and requires TLS when bound off loopback.
- The built-in operation catalog includes `acorn.version`. ACORN has not gone live and no protocol compatibility guarantee has been made, so this plan removes that operation instead of carrying a version contract forward.

# Project code boundaries

- **Create:** `crates/acorn-analysis/`, `crates/acorn-service/`, and `crates/acorn-server/`; focused fixtures and tests; unversioned protobuf definitions for the typed gRPC surface.
- **Update:** workspace manifests and publishing order; analyzer and repository-workflow consumers in `acorn-lib`; `acorn-wasm`; the CLI serve composition; Buf policy; feature, portability, architecture, server, and package documentation; focused CI/Make targets.
- **Move:** portable analysis types and functions into `acorn-analysis`; transport-neutral service requests, responses, handlers, operation definitions, JSON-RPC dispatch, and client conveniences into `acorn-service`; listener, TLS, authentication, health, shutdown, and process configuration into `acorn-server`.
- **Retain:** concrete filesystem traversal, document conversion, Vale runtime, fixes, inference, terminal rendering, persistent database implementation, GitLab/MCP transports, and other native adapters in their current host owners until their dedicated extraction plans land.
- **Delete after cutover:** moved implementations and ownership-masking re-exports from `acorn-lib`; the generic gRPC `Invoke`/`Notify`/`Batch` protocol and its `google.protobuf.Value` conversion layer; the `acorn.version` service operation and its transport/tool exposure.

Before implementation, each phase must replace these directory-level boundaries with an exact file write set and claim the workspace graph, library roots, analyzer contract, RPC/protobuf, CLI contract, CI/build, and documentation-generation locks it needs.

# Dependencies and ordering

- [Dependency capabilities and macOS unwind size](../portability/dependency-features-and-macos-size.md) precedes creation of the new crates and owns the prerequisite workspace feature cleanup.
- This plan is the executable extraction slice authorized by the [crate boundary architecture](../crate-boundary-architecture.md). It does not wait for `acorn-api`, `acorn-store`, or `acorn-export` because its first service catalog accepts supplied content and uses the existing persistence adapter only for retained JSON-RPC operation status/replay.
- The service request and error contracts must stabilize before [CLI composition](cli-composition-boundary.md) consumes them.
- Provider orchestration, artifact rendering, and store extraction remain downstream work. They may add service operations later without moving concrete providers, renderers, or databases into `acorn-service`.
- Phase 1 is one acceptance unit: do not land an empty `acorn-analysis` crate without its service consumer, or a service crate that duplicates analysis logic.

# Phase 1 — extract `acorn-analysis` and `acorn-service` together

## `acorn-analysis` ownership

Move the portable analysis kernel into a crate that depends only on `acorn-core`, `acorn-schema`, and narrow portable libraries.

The crate owns:

- `Check`, `CheckBatch`, categories, severities, summaries, standards, findings, and validation issue/error models;
- validation-error folding and deterministic result aggregation;
- readability metrics, thresholds, supplied-text checks, and their immutable reference data;
- FAIR and supplied-observation checks that do not discover or read host resources;
- portable macros such as `check!`, `check_ok!`, and `check_err!`, rewritten to resolve through their owning crate;
- pure traversal over caller-supplied documents or content collections.

The crate does not own:

- filesystem/path traversal, globbing, source discovery, HTTP, databases, subprocesses, clocks, or environment reads;
- document conversion, terminal labels, tracing side effects, or `OutputFormat`;
- Vale installation, configuration, process execution, or rendered output;
- fix planning/execution, inference, or the semantic diff work owned by the separate diff plan;
- host-only `DocumentTarget::query`; replace it with an extension in the host implementation rather than making paths part of the portable type.

Compile immutable readability and FAIR reference data directly into the crate rather than routing through the generic `acorn-lib` asset runtime. Pure functions return values or typed errors and do not log. Support the workspace's `std`, alloc-only, and `wasm32-unknown-unknown` portability checks without enabling native dependencies.

Migrate first-party imports directly to `acorn_analysis`. Do not retain an `acorn-lib::analyzer` compatibility facade or an obsolete feature that silently enables the new crate.

## `acorn-service` ownership

Create a transport-neutral crate that depends on `acorn-core`, `acorn-schema`, and `acorn-analysis`, not on `acorn-host` or a concrete store. Its initial public application surface is deliberately small:

| Rust operation | JSON-RPC method | gRPC method | Result |
|---|---|---|---|
| `validate_research_activity` | `acorn.research-activity.validate` | `ValidateResearchActivity` | normalized validation diagnostics and validity summary |
| `check_readability` | `acorn.readability.check` | `CheckReadability` | metric, score/checks, and deterministic summary |
| `process_repository` | `acorn.workflows.process` | `ProcessRepository` | ordered repository file reports and proposed changes |

Public request and response structs are the canonical application contract. They use Serde with unknown-field rejection where appropriate, carry optional source identifiers as data rather than paths to read, and return structured diagnostics rather than rendered strings. The transport adapters map to these values; they do not reimplement validation, readability, or workflow policy.

`ServiceError` is a typed, transport-neutral error with stable categories for invalid input, unsupported format or metric, policy denial, payload bounds, unavailable durable operations, and internal failures. It retains safe field paths and operation context but no terminal styling, HTTP status, gRPC status, or `color-eyre` report. Each adapter owns the mapping from these categories to JSON-RPC errors or gRPC status codes.

The service methods are deterministic over their supplied requests:

- `validate_research_activity` decodes through schema-owned string/byte codecs, runs schema validation, and returns every user-facing deserialization/validation diagnostic rather than failing at the first field.
- `check_readability` accepts text, a canonical metric, threshold options, and an optional logical source URI. It does not open files.
- `process_repository` accepts project/revision identifiers, ordered actions, and file path/content/format values. It returns a change proposal and never writes a repository.
- Repository paths receive portable lexical validation: reject empty, absolute, platform-prefixed, parent-traversing, and ambiguous normalized paths. Symlink containment and materialization remain host-adapter responsibilities.
- Formatting and linked-content generation use schema-owned codecs over supplied data. They must not scan the process working directory or hydrate images from implicit paths.
- Results sort by documented keys so native, WASM, JSON-RPC, and gRPC callers receive equivalent ordering.

Retain the transport-neutral portions of the existing operation registry in `acorn-service`: validated method names, typed registration, effect declarations, invocation context, authorization, dispatch, standard JSON-RPC errors, batch ordering, notification semantics, and input/output bounds. Register the three canonical service operations against the same service instance used by direct Rust callers.

Keep `acorn.operations.status` and `acorn.operations.replay` in the JSON-RPC catalog. Express their storage need through a narrow durable-operation port consumed by `acorn-service`; compose the existing persistence adapter outside the crate. Replay remains mutation-gated and startup continues to reject mutation enablement when local persistence is unavailable. Storage extraction itself is outside this plan.

Remove `acorn.version` from the shared operation catalog and from MCP/JSON-RPC exposure that delegates to that catalog. Do not replace it with build metadata, `GetVersion`, a protocol negotiation call, or a protobuf package version.

## Phase 1 consumer cutover

- Point `acorn-wasm` validation and readability bindings at `acorn-analysis`/`acorn-service`, retaining the JavaScript-facing shapes through explicit conversions and parity tests.
- Make existing CLI and host analysis paths consume `acorn-analysis` models while leaving path discovery and rendering native.
- Make GitLab automation, MCP tools, and the existing JSON-RPC adapter call `acorn-service` operations rather than constructing a second repository workflow registry.
- Leave provider, persistence, webhook, and MCP transport mechanics in `acorn-lib`; only their application-operation dependency crosses into `acorn-service`.
- Keep every intermediate commit buildable. Phase 1 completes only when direct Rust, host, WASM, and existing JSON-RPC consumers use the new owners.

# Phase 2 — build the dual-protocol `acorn-server`

## Process and configuration

Create `acorn-server` as a library-and-binary composition root. One process starts JSON-RPC and gRPC on separate listeners and injects one shared `acorn-service` instance plus the existing durable-operation adapter.

- Enable both listeners by default: JSON-RPC on `127.0.0.1:8001` and gRPC on `127.0.0.1:50051`.
- Provide explicit bind flags/environment variables and explicit per-listener disable flags. Reject startup when both listeners are disabled.
- Read the application bearer token from `ACORN_SERVICE_TOKEN`; reject missing or empty secrets and do not accept the token as a command-line value.
- Accept flags and environment variables for TLS certificate/key paths, request/message size, request timeout, graceful-shutdown deadline, listener selection/binds, mutation policy, and tracing level. Do not add a server configuration file in this phase.
- Preserve the current 1 MiB request/message and output limits and 30-second application request timeout as defaults. Apply bounds before expensive decoding or analysis.
- Use the existing ACORN database resolution for durable JSON-RPC status/replay. Do not create a second persistence format.
- Emit structured tracing with protocol, operation, correlation/request identifier, outcome, latency, and bounded size fields. Never log bearer tokens, request bodies, supplied research data, or TLS key material.

Use CLI-over-environment-over-default precedence and keep the process contract explicit:

| Purpose | CLI | Environment | Default |
|---|---|---|---|
| JSON-RPC listener | `--json-rpc-bind` | `ACORN_SERVER_JSON_RPC_BIND` | `127.0.0.1:8001` |
| gRPC listener | `--grpc-bind` | `ACORN_SERVER_GRPC_BIND` | `127.0.0.1:50051` |
| Disable JSON-RPC | `--disable-json-rpc` | `ACORN_SERVER_JSON_RPC_ENABLED=false` | enabled |
| Disable gRPC | `--disable-grpc` | `ACORN_SERVER_GRPC_ENABLED=false` | enabled |
| TLS identity | `--tls-cert`, `--tls-key` | `ACORN_SERVER_TLS_CERT`, `ACORN_SERVER_TLS_KEY` | absent on loopback |
| Request/message bound | `--max-request-bytes` | `ACORN_SERVER_MAX_REQUEST_BYTES` | 1 MiB |
| Application timeout | `--request-timeout-seconds` | `ACORN_SERVER_REQUEST_TIMEOUT_SECONDS` | 30 seconds |
| Shutdown drain | `--shutdown-grace-seconds` | `ACORN_SERVER_SHUTDOWN_GRACE_SECONDS` | 30 seconds |
| Mutation authorization | `--allow-mutation` | `ACORN_SERVER_ALLOW_MUTATION=true` | disabled |
| Trace filter | none | `ACORN_LOG_LEVEL` | existing ACORN default |

`ACORN_SERVICE_TOKEN` has no CLI equivalent. Reject zero values for byte/time bounds, incomplete TLS pairs, invalid booleans or socket addresses, mutation without durable persistence, and any configuration that disables both listeners.

Plaintext is permitted only when every enabled listener is bound to loopback. A non-loopback bind requires a complete TLS identity and applies TLS to that listener. Both application protocols require the same bearer token: HTTP `Authorization: Bearer ...` for JSON-RPC and authorization metadata for gRPC. Health endpoints are unauthenticated and expose no build or configuration details.

## JSON-RPC server

Host the retained JSON-RPC 2.0 adapter in `acorn-server`:

- `POST /rpc` accepts `application/json` requests and uses the shared `acorn-service` registry.
- Preserve single calls, ordered batches, notifications, notification-only `204 No Content`, standard parse/request/method/params errors, effect authorization, correlation IDs, and bounded results.
- Preserve `acorn.operations.status` and mutation-gated `acorn.operations.replay` through the injected durable-operation port.
- Expose the three canonical portable operations named in phase 1 and ensure their successful results and public failures match direct Rust and gRPC behavior.
- Keep HTTP authentication, media-type checks, body limits, timeouts, cancellation, TLS, readiness, and graceful draining in the server adapter rather than the service crate.
- Do not restore `acorn.version`.

Provide a small Rust JSON-RPC client with typed convenience methods for the three canonical operations plus explicit `call`, `notify`, and ordered `batch` entry points. It enforces endpoint scheme, bearer authentication, request timeout, response bounds, JSON-RPC identifier matching, and typed error decoding.

## Typed gRPC server

Replace the generic gRPC envelope with an unversioned protobuf package and explicit service:

```proto
package acorn.service;

service AcornService {
  rpc ValidateResearchActivity(ValidateResearchActivityRequest) returns (ValidateResearchActivityResponse);
  rpc CheckReadability(CheckReadabilityRequest) returns (CheckReadabilityResponse);
  rpc ProcessRepository(ProcessRepositoryRequest) returns (ProcessRepositoryResponse);
}
```

Define explicit messages for all request, diagnostic, check summary, file report, and proposed-change fields. Do not use `google.protobuf.Value`, an `Invoke` method, method-name strings, generic batches, or a service/version RPC. Update Buf lint policy narrowly to permit the unversioned package and `AcornService` name; keep ordinary field, enum, and breaking checks enabled.

Generate an ergonomic Rust client and conversion layer that accepts and returns the canonical `acorn-service` request/response values. Preserve bearer metadata, TLS endpoint validation, 1 MiB encoding/decoding limits, deadlines, and cancellation. The generated wire types are not the application model and do not leak into direct callers.

## Health and shutdown

- Register the standard gRPC health service and expose minimal HTTP `/healthz` and `/readyz` endpoints on the JSON-RPC listener.
- Report liveness once the process runtime is active. Report readiness only after every enabled listener and required durable adapter has initialized.
- On SIGINT or SIGTERM, mark readiness false and gRPC health `NOT_SERVING`, stop accepting new application work, drain in-flight requests up to the configured 30-second default grace period, then cancel remaining work and exit nonzero if shutdown was forced.
- If either enabled listener fails at startup or terminates unexpectedly, take the whole process out of readiness, shut down the peer listener, and return a failure. Do not leave a partially serving daemon.
- Metrics, reflection, mTLS, service discovery, and container packaging are outside this phase.

# Phase 3 — cut over composition roots and remove duplication

- Make `acorn serve rpc` and `acorn serve grpc` thin first-party composition entry points that use the corresponding `acorn-server` listener with existing remote/detached dispatch behavior. They continue to work; JSON-RPC capability is not removed.
- Add a documented `acorn-server` binary invocation for the normal dual-listener deployment. Do not make the main CLI spawn a subprocess for local serving.
- Keep JSON-RPC and gRPC protocol-specific options local to their adapters; share token loading, TLS validation, service construction, persistence setup, tracing, health, and shutdown orchestration.
- Remove the old `acorn-lib` Axum listener, generic gRPC server/client/protobuf envelope, duplicated operation construction, and obsolete feature wiring after parity tests pass.
- Regenerate CLI help, protobuf artifacts, package documentation, dependency diagrams, and publishing scripts in topological order.
- Update examples for direct Rust embedding, JSON-RPC calls, typed gRPC calls, and one dual-protocol mini-service deployment.

# Validation

## Portable analysis and service

- Unit tests preserve current check/category/severity/summary behavior, readability scores and thresholds, validation folding, reference-data results, and deterministic ordering.
- Compile `acorn-analysis` and the core of `acorn-service` under default, no-default/alloc, and `wasm32-unknown-unknown` configurations; inspect their feature trees for filesystem, HTTP server, database, process, terminal, tracing-subscriber, and native TLS leakage.
- Service tests cover valid and malformed RAD input, multiple deserialization/validation failures, every readability metric, unsupported metrics/formats, output bounds, deterministic repository ordering, duplicate paths, unsafe lexical paths, format/link actions, and no current-directory image discovery.
- Direct Rust, WASM, JSON-RPC, and gRPC golden cases produce equivalent canonical results after adapter conversion.

## JSON-RPC

- Preserve the existing single-call, invalid-document, batch-order, notification omission, effect-policy, output-limit, workflow-equivalence, status, and replay tests under the new owner.
- Add HTTP tests for missing/wrong bearer tokens, media type, request/body/output bounds, deadline cancellation, TLS enforcement off loopback, health authentication policy, readiness transitions, and graceful drain.
- Test typed client calls, generic calls, notifications, mixed batches, identifier mismatch, malformed responses, response bounds, timeouts, and TLS endpoint policy.
- Assert `acorn.version` returns method-not-found and is absent from exported tool/operation catalogs.

## gRPC and daemon

- Test each typed RPC over an in-process server for success, every public error category, malformed protobuf values, maximum messages, deadlines, client cancellation, bearer authentication, TLS, and conversion parity.
- Test standard gRPC health and HTTP health/readiness without authentication, while confirming application endpoints always require authentication.
- Start the dual-listener binary on ephemeral ports and prove both protocols share one service configuration and persistence adapter, either-listener failure shuts down the process, signals drain both listeners, and forced timeout returns failure.
- Keep focused protocol tests hermetic; use loopback sockets and generated test certificates without relying on external services.

## Repository gates

- Run focused package tests after each slice, then formatting, Clippy/lint, the full test suite, WASM portability, documentation, generated-help drift, protobuf lint/breaking checks, dependency inspection, and package dry-runs in dependency order.
- Smoke-test direct library embedding, `acorn-server` with both listeners, `acorn serve rpc`, and `acorn serve grpc`.

# Acceptance criteria

- `acorn-analysis` owns the portable analysis kernel and compiles without native host capabilities.
- `acorn-service` exposes the three canonical operations over supplied data, depends on no concrete host/store adapter, and returns typed deterministic results and errors.
- Native and WASM consumers import the new owning crates directly; `acorn-lib` has no compatibility re-export hiding moved ownership.
- Repository processing performs portable lexical path validation, has no implicit filesystem reads or writes, and cannot inspect the current working directory for images.
- One `acorn-server` process can serve authenticated JSON-RPC and typed gRPC concurrently, or either listener explicitly, with shared policy and service behavior.
- JSON-RPC retains calls, ordered batches, notifications, effect checks, durable status/replay, request/output bounds, and `acorn serve rpc` support.
- gRPC exposes only explicit typed application RPCs and no generic JSON-value invocation envelope.
- Non-loopback application traffic requires TLS; application methods require bearer authentication; health endpoints reveal no sensitive data.
- Health/readiness and graceful shutdown cover both listeners without leaving a partially serving process.
- No service or transport version operation/package is introduced, and `acorn.version` is removed from the shared catalog.
- Full lint, tests, WASM checks, documentation generation, protobuf checks, and package dry-runs pass.

# Deferred scope

- Provider clients, GitLab webhook transport, MCP transport, artifact rendering/export, and concrete database extraction into their planned crates.
- Filesystem-backed analysis discovery, Vale execution, automatic fixes, inference, and semantic diff behavior.
- Metrics, tracing exporters, gRPC reflection, mTLS/client certificates, service discovery, config files, container images, deployment manifests, and compatibility/version negotiation.
- Additional service operations until a concrete mini-service consumer establishes their request, authorization, and resource-bound contracts.

# Related plans

- [Crate boundary architecture](../crate-boundary-architecture.md)
- [Dependency capabilities and macOS unwind size](../portability/dependency-features-and-macos-size.md)
- [CLI composition boundary](cli-composition-boundary.md)
