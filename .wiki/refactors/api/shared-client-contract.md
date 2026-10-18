---
okf_version: "0.1"
type: Plan
title: Complete the shared API client contract
description: Finish endpoint, request, invocation, configuration, error, health, and OpenAPI boundaries before extracting provider clients.
difficulty: high
impact: high
package: [acorn-lib, acorn-schema]
tags: [refactor, api, endpoints, requests, configuration, errors, health, raid]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Finish one transport-neutral API client contract that provider modules can use without duplicating endpoint construction, parameter validation, environment discovery, send/status/body/parse sequences, health interpretation, or error extraction. Use the completed contract to harden the RAiD provider and prepare a clean `acorn-api` extraction.

# Current evidence

- `Endpoint::{from_template,with_domain,resource_url}` and immutable `Params` builders are implemented and used by some providers.
- GitLab authentication uses shared `Authenticator` and `Credential` types with deterministic token precedence and header selection.
- Many providers still perform their own endpoint lookup/domain substitution or construct raw `Vec<Param>` values.
- Provider functions repeat `invoke`/`invoke_with` followed by `handle`/`handle_or`, while ORCID, ROR, CiteAs, and Harbor expose similar but separate health/status handling.
- RAiD already has typed service-point and error responses. SWHID resolver metadata and Handle value payloads remain open JSON; no current call-site evidence justifies a shared response DTO phase. OpenAI intentionally retains broad response JSON for evolving payloads.
- RAiD already exposes service-point and record list/lookup/create operations, but the useful workflow evidence from the removed gather prototype is not captured by fixture-backed service tests.
- `InvocationContext`, `OperationEffects`, and shared authorization now live in `io::api::operation` and are reused by the outbound MCP client. That operation boundary is implemented and is distinct from the provider HTTP invocation helpers still planned here.

# Project code boundaries

- **Update:** shared API and configuration roots, applicable provider modules, endpoint catalog fixtures, RAiD schema adapters, and focused provider tests; later the corresponding `acorn-api` extraction surface.
- **Create:** an immutable resolved API configuration input, narrow invocation/response metadata helpers, provider-owned error envelopes, endpoint/OpenAPI validation, and RAiD workflow fixtures.
- **Delete:** migrated endpoint, parameter, environment, send/parse, status, and untyped error-body duplication. Do not delete provider-specific policy or broad JSON retained for documented compatibility.

# Dependencies

Preserve the landed `SecretString`, authentication, bounded HTTP, redirect, redaction, offline, webhook, and operation-authorization contracts. Shared `io/api/mod.rs`, `io/config/mod.rs`, application endpoint snapshots, and the outbound MCP configuration shape are integration locks; provider leaves may migrate independently only after the shared contract for that slice is frozen.

# Decisions

- Shared helpers own mechanics; provider modules retain endpoint names, parameter names, success predicates, DTOs, and fallback policy.
- A helper is added only when at least two current call sites share the same behavior. Do not create a generic client framework around hypothetical providers.
- Required identifiers, bodies, and secrets fail before transport. Optional query/body values are omitted when absent or empty.
- Invocation helpers take an injected transport and make fallback behavior visible in their signatures. Health helpers classify transport-neutral response metadata through a caller-supplied provider predicate.
- Environment reads occur once at host composition and produce immutable resolved configuration with explicit precedence. Provider operations do not call `dotenv` or process environment directly.
- Error envelopes preserve provider details for diagnostics while exposing one redaction-aware message extraction boundary.
- OpenAPI validation checks configured operation names and required template keys before the first request. Code generation remains outside this plan.
- Keep the implemented operation-effects authorization contract separate from provider HTTP invocation helpers; do not move or redesign it incidentally during API extraction.

# Implementation sequence

1. Finish endpoint and required/optional parameter migration across providers.
2. Add the narrow invocation/parse and response-status contracts and migrate repeated call shapes.
3. Resolve provider environment/configuration once and remove provider-local environment reads.
4. Add health helpers and error envelopes only for current duplicate or untyped consumers.
5. Validate OpenAPI-backed endpoint templates against imported operations.
6. Rebase RAiD list, lookup, and create workflows onto the finished contract and add offline fixtures plus ignored sandbox write tests.

Each slice lands with focused provider tests before another shared abstraction is introduced.

# Validation

- Preserve application endpoint parsing, built-in endpoint snapshots, configuration compatibility, and token precedence tests.
- Test required/optional parameters, explicit fallbacks, malformed responses, redaction, health predicates, and missing endpoint/template keys.
- Verify providers make no environment read or request before resolved configuration and input validation succeed.
- RAiD fixtures cover service-point and record list/lookup, metadata validation before create, authentication failures, and field-level API errors. Live writes remain ignored and require explicit sandbox credentials.
- Run provider-focused tests after each leaf migration, then config tests, lint, the full suite, portability checks, and dependency-tree inspection.

# Acceptance criteria

- Applicable providers use one endpoint rendering and required/optional parameter boundary.
- Repeated send/status/body/deserialize paths use a narrow shared helper without hiding provider parsing or fallback choices.
- Provider clients consume immutable resolved configuration and no longer load dotenv or discover secrets independently.
- Health handling shares transport classification while keeping provider-specific success rules explicit.
- Error messages are typed and redaction-aware; evolving response payloads are not made artificially strict.
- Missing OpenAPI operations or template keys fail before network access.
- RAiD list, lookup, and create workflows are fixture-backed, validate metadata before writes, and cannot target production accidentally from ordinary tests.
- The resulting ownership is acyclic and ready to move into `acorn-api` without moving workflow orchestration or persistence with it.

# Execution phases

## API-01 — Inventory remaining client duplication

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-lib/src/io/api/tests/mod.rs`, `.wiki/refactors/api/shared-client-contract.md`
- Primary integration lock: shared API contract
- Outcome: Freeze remaining endpoint, parameter, invocation, environment, health, DTO, and error duplication by provider.
- Focused validation: Run existing provider tests and record exact callers for each later phase.
- Handoff: Remove API-01 and make only the first provider-family phase ready.

## API-02 — Migrate identity-provider endpoint composition

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-01
- Exact write set: `crates/acorn-lib/src/io/api/orcid.rs`, `crates/acorn-lib/src/io/api/ror.rs`, `crates/acorn-lib/src/io/api/isni.rs`, `crates/acorn-lib/src/io/api/handle.rs`, `crates/acorn-lib/src/io/api/orcid/tests.rs`, `crates/acorn-lib/src/io/api/tests/mod.rs`
- Primary integration lock: shared API contract
- Outcome: Use shared endpoints and immutable required/optional parameters for the identity providers.
- Focused validation: Run focused ORCID, ROR, ISNI, and Handle tests.
- Handoff: Remove API-02 and record any reusable call shape for API-05.

## API-03 — Migrate research-metadata endpoint composition

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-01
- Exact write set: `crates/acorn-lib/src/io/api/citeas.rs`, `crates/acorn-lib/src/io/api/openalex.rs`, `crates/acorn-lib/src/io/api/osti.rs`, `crates/acorn-lib/src/io/api/swhid.rs`, `crates/acorn-lib/src/io/api/tests/mod.rs`, `crates/acorn-lib/src/io/api/osti/tests.rs`
- Primary integration lock: shared API contract
- Outcome: Use shared endpoint and parameter mechanics while preserving provider fallback policy.
- Focused validation: Run focused CiteAs, OpenAlex, OSTI, and SWHID tests.
- Handoff: Remove API-03 and record repeated invocation shapes.

## API-04 — Migrate artifact and model endpoint composition

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-01
- Exact write set: `crates/acorn-lib/src/io/api/github/mod.rs`, `crates/acorn-lib/src/io/api/gitlab/service.rs`, `crates/acorn-lib/src/io/api/harbor.rs`, `crates/acorn-lib/src/io/api/huggingface.rs`, `crates/acorn-lib/src/io/api/models_dev.rs`, `crates/acorn-lib/src/io/api/github/tests.rs`, `crates/acorn-lib/src/io/api/gitlab/tests/mod.rs`, `crates/acorn-lib/src/io/api/harbor/tests.rs`, `crates/acorn-lib/src/io/api/tests/mod.rs`
- Primary integration lock: shared API contract
- Outcome: Remove local template/domain and raw-parameter construction from artifact/model providers.
- Focused validation: Run focused GitHub, GitLab, Harbor, Hugging Face, and models.dev tests.
- Handoff: Remove API-04 and record shared mechanics for API-05.

## API-05 — Add narrow invocation and parsing helpers

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-02, API-03, API-04
- Exact write set: `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/io/config/application.rs`, `crates/acorn-lib/src/io/api/tests/mod.rs`
- Primary integration lock: shared API contract
- Outcome: Centralize only duplicated injected-transport invocation, parsing, and explicit fallback shapes.
- Focused validation: Run helper and migrated-provider success/error/fallback tests.
- Handoff: Remove API-05 and freeze the minimal helper surface.

## API-06 — Resolve provider configuration once

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-04
- Exact write set: `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/io/api/tests/mod.rs`
- Primary integration lock: API host composition
- Outcome: Define immutable resolved provider configuration with explicit precedence at host composition.
- Focused validation: Test precedence, missing secrets, empty values, redaction, and configuration immutability.
- Handoff: Remove API-06 and pass the resolved contract to API-14.

## API-07 — Unify health and status classification

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-05
- Exact write set: `crates/acorn-lib/src/io/api/localhost.rs`, `crates/acorn-lib/src/io/api/orcid.rs`, `crates/acorn-lib/src/io/api/ror.rs`, `crates/acorn-lib/src/io/api/harbor.rs`, `crates/acorn-lib/src/io/api/tests/mod.rs`, `crates/acorn-lib/src/io/api/orcid/tests.rs`, `crates/acorn-lib/src/io/api/harbor/tests.rs`
- Primary integration lock: shared API contract
- Outcome: Classify transport-neutral response metadata through provider-owned success predicates.
- Focused validation: Run health/status tests for success, degraded, transport, and malformed responses.
- Handoff: Remove API-07 and preserve provider-specific predicates.

## API-09 — Normalize redacted error envelopes

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-05
- Exact write set: `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/io/api/tests/mod.rs`
- Primary integration lock: shared API contract
- Outcome: Preserve provider detail while exposing one redaction-aware diagnostic message boundary.
- Focused validation: Test secret removal, nested messages, missing detail, and transport failures.
- Handoff: Remove API-09 and document error semantics for consumers.

## API-10 — Validate OpenAPI templates before transport

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-02, API-03, API-04
- Exact write set: `crates/acorn-lib/src/io/api/openapi.rs`, `crates/acorn-lib/src/io/api/tests/mod.rs`
- Primary integration lock: endpoint catalog
- Outcome: Reject missing operations and required template keys before the first request.
- Focused validation: Run imported-operation, missing-key, duplicate, and snapshot tests.
- Handoff: Remove API-10 and record the accepted catalog contract.

## API-11 — Rebase RAiD list and lookup workflows

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-05, API-06, API-09
- Exact write set: `crates/acorn-lib/src/io/api/raid.rs`, `tests/fixtures/raid/read.json`
- Primary integration lock: RAiD provider
- Outcome: Use the completed shared contract for list and lookup without changing workflow semantics.
- Focused validation: Run offline list/lookup fixtures, pagination, not-found, and redaction cases.
- Handoff: Remove API-11 and reuse the contract for create.

## API-12 — Rebase and guard RAiD creation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-11
- Exact write set: `crates/acorn-lib/src/io/api/raid.rs`, `crates/acorn-lib/src/io/api/tests/mod.rs`
- Primary integration lock: RAiD provider
- Outcome: Validate metadata before writes and make production targeting impossible in ordinary tests.
- Focused validation: Run offline request fixtures and explicitly ignored sandbox write tests.
- Handoff: Remove API-12 and record the safe live-test invocation.

## API-14 — Migrate provider environment discovery

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-06
- Exact write set: `crates/acorn-lib/src/io/api/harbor.rs`, `crates/acorn-lib/src/io/api/openalex.rs`, `crates/acorn-lib/src/io/api/citeas.rs`, `crates/acorn-lib/src/io/api/raid.rs`, `crates/acorn-lib/src/io/api/tests/mod.rs`, `crates/acorn-lib/src/io/api/harbor/tests.rs`
- Primary integration lock: API host composition
- Outcome: Pass resolved configuration to provider operations and remove operation-time environment reads from the identified provider call paths.
- Focused validation: Test precedence, missing secrets, redaction, and identical provider requests with environment changes after composition.
- Handoff: Remove API-14 and record any remaining provider-local environment read found by API-01 as a separate bounded phase.

## API-13 — Close the shared client contract

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: API-07, API-10, API-12, API-14
- Exact write set: `docs/src/packages/acorn-api.md`, `.wiki/index.md`, `.wiki/refactors/api/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/refactors/api/shared-client-contract.md`
- Primary integration lock: API documentation
- Outcome: Run full gates, document ownership, and remove the plan before crate extraction.
- Focused validation: Run focused provider suites, portability, `make lint`, `make test`, `make doc`, and wiki validation.
- Handoff: Delete this plan and every index, network, and lane entry.
