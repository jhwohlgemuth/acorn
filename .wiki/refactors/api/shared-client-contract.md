---
okf_version: "0.1"
type: Plan
title: Complete the shared API client contract
description: Finish endpoint, request, invocation, configuration, response, error, health, and OpenAPI boundaries before extracting provider clients.
difficulty: high
impact: high
package: [acorn-lib, acorn-schema]
tags: [refactor, api, endpoints, requests, configuration, errors, dto, health, raid]
status: in-progress
---

# Goal

Finish one transport-neutral API client contract that provider modules can use without duplicating endpoint construction, parameter validation, environment discovery, send/status/body/parse sequences, health interpretation, or error extraction. Use the completed contract to harden the RAiD provider and prepare a clean `acorn-api` extraction.

# Current evidence

- `Endpoint::{from_template,with_domain,resource_url}` and immutable `Params` builders are implemented and used by some providers.
- GitLab authentication uses shared `Authenticator` and `Credential` types with deterministic token precedence and header selection.
- Many providers still perform their own endpoint lookup/domain substitution or construct raw `Vec<Param>` values.
- Provider functions repeat `invoke`/`invoke_with` followed by `handle`/`handle_or`, while ORCID, ROR, CiteAs, and Harbor expose similar but separate health/status handling.
- OpenAI intentionally retains broad response JSON for evolving payloads; other providers consume stable fields that can be represented by narrow DTOs.
- RAiD already exposes service-point and record list/lookup/create operations, but the useful workflow evidence from the removed gather prototype is not captured by fixture-backed service tests.
- `InvocationContext`, `OperationEffects`, and shared authorization now live in `io::api::operation` and are reused by the outbound MCP client. That operation boundary is implemented and is distinct from the provider HTTP invocation helpers still planned here.

# Project code boundaries

- **Update:** shared API and configuration roots, applicable provider modules, endpoint catalog fixtures, RAiD schema adapters, and focused provider tests; later the corresponding `acorn-api` extraction surface.
- **Create:** an immutable resolved API configuration input, narrow invocation/response metadata helpers, provider-owned error envelopes and stable-field DTOs, endpoint/OpenAPI validation, and RAiD workflow fixtures.
- **Delete:** migrated endpoint, parameter, environment, send/parse, status, and untyped error-body duplication. Do not delete provider-specific policy or broad JSON retained for documented compatibility.

# Dependencies

Preserve the landed `SecretString`, authentication, bounded HTTP, redirect, redaction, offline, webhook, and operation-authorization contracts. Shared `io/api/mod.rs`, `io/config/mod.rs`, application endpoint snapshots, and the outbound MCP configuration shape are integration locks; provider leaves may migrate independently only after the shared contract for that slice is frozen.

# Decisions

- Shared helpers own mechanics; provider modules retain endpoint names, parameter names, success predicates, DTOs, and fallback policy.
- A helper is added only when at least two current call sites share the same behavior. Do not create a generic client framework around hypothetical providers.
- Required identifiers, bodies, and secrets fail before transport. Optional query/body values are omitted when absent or empty.
- Invocation helpers take an injected transport and make fallback behavior visible in their signatures. Health helpers classify transport-neutral response metadata through a caller-supplied provider predicate.
- Environment reads occur once at host composition and produce immutable resolved configuration with explicit precedence. Provider operations do not call `dotenv` or process environment directly.
- Type only stable consumed response fields. Keep evolving or unconsumed fields optional or as raw JSON rather than overfitting full remote schemas.
- Error envelopes preserve provider details for diagnostics while exposing one redaction-aware message extraction boundary.
- OpenAPI validation checks configured operation names and required template keys before the first request. Code generation remains outside this plan.
- Keep the implemented operation-effects authorization contract separate from provider HTTP invocation helpers; do not move or redesign it incidentally during API extraction.

# Implementation sequence

1. Finish endpoint and required/optional parameter migration across providers.
2. Add the narrow invocation/parse and response-status contracts and migrate repeated call shapes.
3. Resolve provider environment/configuration once and remove provider-local environment reads.
4. Add health helpers, stable response DTOs, and error envelopes only for current duplicate or untyped consumers.
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
- Stable consumed fields and error messages are typed and redaction-aware; evolving payloads are not made artificially strict.
- Missing OpenAPI operations or template keys fail before network access.
- RAiD list, lookup, and create workflows are fixture-backed, validate metadata before writes, and cannot target production accidentally from ordinary tests.
- The resulting ownership is acyclic and ready to move into `acorn-api` without moving workflow orchestration or persistence with it.
