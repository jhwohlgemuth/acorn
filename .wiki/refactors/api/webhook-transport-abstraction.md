---
type: Plan
title: "Extract reusable webhook transport and delivery contracts"
description: "Move shared webhook verification, delivery metadata, and outbound JSON dispatch behind acorn::io::api::webhooks while retaining provider-specific adapters."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [refactor, api, webhook, gitlab, integration, power-automate]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/io/http/mod.rs`, `crates/acorn-lib/src/io/api/gitlab/{mod,webhook,service,bot}.rs`, their focused tests, `crates/acorn-cli/src/commands/{create,serve}/bot/mod.rs`, and webhook-facing documentation.
- **Create:** `crates/acorn-lib/src/io/api/webhooks/` with public delivery, authentication, Standard Webhooks, outbound-dispatch, and focused test modules.
- **Delete:** None. Keep `crates/acorn-lib/src/io/api/gitlab/webhook.rs` as the GitLab event adapter and preserve provider-facing compatibility aliases during the extraction.

# Dependencies

Land [GitLab claimed-operation capabilities](gitlab-claimed-operation-capabilities.md) first so this refactor consumes a stable queue-settlement contract without moving provider workflow state into the generic module. This plan then precedes the remaining [GitLab bot workflows](gitlab-webhook-bot-workflows.md) and any Power Automate-specific feature or configuration surface.

The implementation owns the `io/api/mod.rs` module-index lock and the `io/http/mod.rs` HTTP-transport lock. It must not overlap endpoint, request-parameter, or HTTP-policy refactors on those files. The initial in-tree path is `acorn::io::api::webhooks`; later crate extraction can move pure protocol contracts to `acorn-api` and concrete Axum, HTTP-client, and persistence adapters to their host, service, and store owners.

# Goal

Expose a reusable `acorn::io::api::webhooks` boundary for authenticated inbound deliveries and safe outbound JSON callbacks. Refactor the GitLab bot to consume that boundary without changing its routes, accepted payloads, authentication behavior, durable operation semantics, or project-hook registration behavior. Prove that a non-GitLab caller can deliver a typed event to a Power Automate-style HTTPS callback URL without importing GitLab types or exposing URL-embedded credentials.

# Current evidence

- `crates/acorn-lib/src/io/api/gitlab/webhook.rs` combines reusable mechanics with GitLab policy: raw-body limits, shared-secret comparison, Standard Webhooks HMAC verification, replay-window checks, delivery-ID extraction, GitLab header and payload decoding, project validation, canonical operation keys, queue persistence, and default bot handling.
- `WebhookDelivery`, `WebhookOperation`, and `WebhookOperationHandler` are exported from `io::api::gitlab` even though delivery envelopes and asynchronous handler shapes are not inherently GitLab-specific.
- `WebhookOptions` combines an externally reachable endpoint, legacy inbound secret, Standard Webhooks signing secret, GitLab-version negotiation, and GitLab registration URL construction.
- GitLab project-hook list/create/update calls belong to the GitLab provider API. They should consume shared webhook endpoint and authentication values, not move into the generic module.
- `OperationQueue` and its database tables encode bot operations, retries, settlement, and GitLab workflow payloads. Persistence is a service/store concern and should remain behind a narrow sink or queue boundary rather than become part of the transport abstraction.
- `io::http` can POST arbitrary JSON through an injected `HttpService`, but request display and tracing currently include the full URL. Power Automate and Logic Apps request-trigger URLs can contain an SAS signature in their query string, so full callback URLs must be treated as credentials and redacted from diagnostics.[4]
- The shared HTTP policy does not retry POST requests. That is a safe default for callbacks because an automatic retry can duplicate downstream workflow execution unless the receiver enforces idempotency.
- Standard Webhooks defines signed delivery ID, timestamp, raw payload, secret rotation, replay prevention, and idempotency metadata independently of any provider payload shape.[1] GitLab 19 uses this scheme while retaining its legacy secret-token path.[2]
- Power Automate can start a flow from a generated HTTP request endpoint and can restrict callers with Microsoft Entra authentication.[3] A later integration may supply a bearer token or an opaque SAS-bearing URL; token acquisition and flow provisioning are separate concerns.

# Decisions

## Generic ownership

- Model a typed delivery independently of its provider payload, conceptually `Delivery<T> { id, event }`. Keep the raw request body available through verification and decoding so signature checks happen before JSON parsing or reserialization.
- Define small authentication/verifier contracts. The first implementations are Standard Webhooks HMAC-SHA256 and constant-time comparison of a configured secret header. Provider adapters choose header names and authentication precedence.
- Normalize authentication, size-limit, stale-timestamp, malformed-delivery, and downstream-status failures into a webhook-owned error taxonomy that adapters can map to HTTP responses or `ApiResult` reports.
- Provide outbound typed JSON dispatch over an injected `HttpService`. The target owns the complete callable URL and optional headers or caller-supplied bearer token; the generic module does not acquire OAuth tokens or understand provider query parameters.
- Treat callback URLs and credential headers as secrets. Diagnostics may show scheme and authority plus a redacted path/query summary, but must never render SAS signatures, bearer tokens, shared secrets, or full opaque callback URLs.
- Perform one outbound POST per dispatch call. Do not add hidden POST retries. A durable outbox, stable delivery identifier, retry schedule, and receiver-specific idempotency policy require a separate service/store design.

## Provider ownership

- Keep GitLab event headers, raw DTOs, `HookPayload`, `HookActor`, merge-request actions, project-ID validation, actionable-operation keys, default analysis/intake handlers, and project-hook registration in `io::api::gitlab`.
- Keep GitLab's legacy-versus-signing-token version negotiation in its registration adapter. The generic module verifies a selected strategy; it does not query GitLab versions.
- Keep the concrete durable queue and settlement lifecycle out of `io::api::webhooks`. The GitLab receiver passes an authenticated, decoded delivery to its existing queue boundary.
- Preserve source compatibility where practical with GitLab aliases such as `WebhookDelivery = webhooks::Delivery<HookPayload>` and provider-specific handler aliases. Do not preserve duplicate implementations.

## Explicit non-goals

- Do not create, update, export, or deploy Power Automate flows.
- Do not implement Microsoft Entra OAuth token acquisition, refresh, tenant discovery, managed identity, or Power Platform connector registration.
- Do not generalize GitLab project-hook registration into a false cross-provider subscription API. Subscription and unsubscription protocols remain adapter-specific.
- Do not move bot operation persistence into the API transport module or rename existing database tables in this refactor.
- Do not add automatic fan-out, scheduling, dead-letter queues, or a general event bus.

# Public interfaces and data contracts

The exact names are finalized by the contract-first implementation slice, but the public surface must cover these roles without exposing Axum or reqwest concrete types in the core contracts:

- A generic typed delivery envelope with a stable delivery identifier.
- An inbound request view containing immutable headers and exact raw bytes.
- A verifier contract plus Standard Webhooks and configured-header-secret implementations.
- A decoder/adapter boundary that turns an authenticated request into `Delivery<T>` or an explicit unsupported-event result.
- A secret-aware HTTPS webhook target and typed outbound event/request.
- A dispatch result that preserves HTTP status and safe response metadata without logging secret URLs or response credentials.

GitLab retains `HookPayload` as its event type and maps the generic errors to its existing `400`, `401`, `403`, `413`, `503`, `202`, and ignored-event responses. Provider registration continues to return `ProjectWebhook` and `WebhookRegistration`.

# Implementation steps

Steps are ordered by dependency. Each step must meet its verification gate before the next step changes provider behavior.

| Order | Impact | Difficulty | Step | Verification gate |
|---:|---|---|---|---|
| 1 | high | medium | Characterize the current GitLab ingress and registration contract. Add or retain tests for legacy tokens, Standard Webhooks signatures and rotation, stale timestamps, tampered raw bodies, missing delivery IDs, payload limits, unexpected projects, unsupported events, duplicate deliveries/operations, queue failures, and create-versus-update registration. | Focused GitLab webhook, queue, bot, and registration tests pass before extraction. |
| 2 | high | high | Introduce `io::api::webhooks` with immutable delivery/request types, verifier contracts, Standard Webhooks verification, constant-time configured-header verification, size and replay policies, and typed errors. Keep verification pure over headers, raw bytes, configured secrets, and an injected/current timestamp. | Unit tests use published signature vectors plus tampering, multiple-signature rotation, stale/future timestamps, invalid secret formats, malformed headers, and exact raw-body preservation. |
| 3 | high | high | Refactor the GitLab receiver to authenticate through the shared verifiers and decode into `Delivery<HookPayload>`. Keep GitLab normalization, project checks, operation keys, queue persistence, default handlers, routes, and registration in the provider adapter. Add compatibility aliases rather than parallel delivery implementations. | Existing GitLab tests pass unchanged where behavior is stable; compile tests cover old GitLab imports and the new generic path; no generic module imports GitLab. |
| 4 | high | medium | Add secret-aware outbound JSON dispatch using `HttpService`. Preserve opaque query parameters on the wire, support optional safe headers or a caller-supplied bearer token, classify success/non-success responses, and prevent full target URLs from reaching display, debug, tracing, or error text. Do not automatically retry POST. | A mock Power Automate-style request endpoint receives the exact JSON and query string once; tests prove `sig`, bearer, and custom secret values never appear in formatted requests, captured logs, or errors, including transport and non-2xx failures. |
| 5 | medium | medium | Migrate bot configuration and documentation to compose generic endpoint/authentication values with GitLab registration policy. Document manual versus API registration, generic outbound usage, security boundaries, and the follow-on Power Automate adapter scope. | CLI parsing and container-argument tests pass; rustdoc builds; generated help changes only if user-facing arguments change. |
| 6 | medium | low | Reconcile crate-extraction ownership and public re-exports after the in-tree contract is green. | `cargo check` succeeds for default and supported reduced feature sets; the API roadmap, dependency network, and GitLab workflow plan match the landed boundary. |

# Validation

1. Run `cargo test -p acorn-lib io::api::webhooks --lib` for generic delivery, verification, dispatch, and redaction behavior.
2. Run the focused GitLab webhook, bot, queue, registration, and intake tests to prove behavioral parity and durable deduplication.
3. Run focused CLI bot parsing, local serve, detached container-argument, and credential-redaction tests.
4. Use local mock servers for both inbound GitLab delivery and outbound Power Automate-style callback scenarios; no live GitLab, Microsoft tenant, or network credential is required for the automated suite.
5. Run supported feature checks, `cargo fmt --check`, `make lint`, `make test`, rustdoc/link validation, and `git diff --check`.
6. Perform one staging GitLab delivery after automated parity is green. Treat a live Power Automate flow as follow-on integration validation, not an acceptance dependency for this abstraction.

# Acceptance criteria

- `acorn::io::api::webhooks` is public and usable without importing GitLab types.
- GitLab webhook receipt, authentication, payload normalization, project isolation, durable enqueueing, deduplication, worker handling, and hook registration retain their tested behavior.
- Standard Webhooks verification signs and verifies the exact raw body with delivery ID and timestamp, accepts valid rotated signatures, uses constant-time verification, and rejects stale or tampered deliveries.[1]
- GitLab-specific payloads, registration APIs, version checks, operation keys, and business handlers do not leak into the generic module.
- A typed non-GitLab event can be POSTed once to an opaque Power Automate-style HTTPS callback through an injected HTTP service.
- Callback query secrets, bearer tokens, shared secrets, signing keys, and credential headers cannot appear in display output, debug output, tracing fields, or returned error messages.
- Outbound POST retry and durable outbox semantics are explicit follow-on policy rather than hidden transport behavior.
- No Power Automate-specific dependency, tenant configuration, flow-management API, or OAuth implementation is introduced by this refactor.
- The planned `acorn-api`/`acorn-service`/`acorn-store` ownership split remains acyclic: protocol types and pure verification can move independently of HTTP servers, clients, bot orchestration, and persistence.

# Follow-on work

- Add a Power Automate integration plan only after a concrete workflow is selected. It can define an ACORN event schema, configuration source, OAuth or SAS policy, delivery/outbox requirements, and end-to-end flow fixture while consuming this generic module.
- Consider producer-side Standard Webhooks signing when ACORN must authenticate outbound events to receivers it controls.
- Consider a durable generic outbox only when at least two outbound integrations share retry, fan-out, settlement, and observability requirements.

# Citations

[1] [Standard Webhooks specification](https://github.com/standard-webhooks/standard-webhooks/blob/main/spec/standard-webhooks.md) - Provider-independent payload, signed metadata, raw-body verification, rotation, replay prevention, and idempotency guidance.

[2] [GitLab webhook signing tokens](https://docs.gitlab.com/user/project/integrations/webhooks/#signing-tokens) - GitLab's Standard Webhooks headers, HMAC calculation, timestamp handling, and legacy-token compatibility.

[3] [Power Automate HTTP trigger authentication](https://learn.microsoft.com/en-us/power-automate/oauth-authentication) - Generated request-trigger endpoints and Microsoft Entra caller restrictions.

[4] [Secure request-trigger access in Azure Logic Apps](https://learn.microsoft.com/en-us/azure/logic-apps/logic-apps-securing-a-logic-app) - SAS-bearing callback URLs, HTTPS requirements, secret handling, revocation, and OAuth alternatives used by the underlying workflow platform.

[5] [Power Automate and Logic Apps custom webhook triggers](https://learn.microsoft.com/en-us/connectors/custom-connectors/create-webhook-trigger) - Provider-specific subscribe and unsubscribe contracts that remain outside the generic delivery transport.
